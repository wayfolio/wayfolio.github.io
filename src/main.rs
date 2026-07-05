use {
    crate::repos::{Handling, REPOS, Repo},
    anyhow::{Context, Error, Result, bail},
    isnt::std_1::collections::IsntHashSetExt,
    regex::Regex,
    rusqlite::{Connection, params},
    std::{
        cell::RefCell,
        collections::{HashMap, HashSet},
        path::Path,
    },
    wayfolio::{
        ast::{
            Arg, ArgType, Copyright, Description, Entry, Enum, Interface, Member, MemberType,
            Message, MessageType, Protocol, Suite,
        },
        compendium::Compendium,
        cs::parser::CompositorSupport,
        site::generate_site,
    },
};

mod repos;

const COMPENDIUM: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/submodules/wayfolio/wasm/src/compendium_src.rs"
);

fn main() -> Result<()> {
    let compositor_support = read_compositor_support()?;
    let compositor_support = compositor_support.get();
    let mut suites = read_protocols()?;
    for suite in &mut suites {
        suite.protocols.sort_by(|l, r| l.name.cmp(&r.name));
    }
    suites[2..].sort_by(|l, r| l.name.cmp(&r.name));
    let suites: Vec<_> = suites.iter().collect();
    let protocols: Vec<_> = suites.iter().flat_map(|s| &s.protocols).collect();
    let compendium =
        Compendium::new(&protocols, &compositor_support).context("create compendium")?;
    std::fs::write(COMPENDIUM, compendium.create_src()).context("write compendium_src.rs")?;
    generate_site(Path::new("page"), &compendium, &suites, &[]).context("generate site")?;
    Ok(())
}

fn read_compositor_support() -> Result<CompositorSupport> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/submodules/compositor-support");
    CompositorSupport::parse(path).context("parse compositor-support")
}

fn read_protocols() -> Result<Vec<Suite>> {
    let name_regex = Regex::new("[^a-zA-Z0-9_]")?;
    let db = Connection::open("./submodules/wayland-db/wayland.db").context("open database")?;
    let prepare = |s| {
        db.prepare(s)
            .with_context(|| format!("Could not prepare {s}"))
    };
    let mut types = HashMap::new();
    // language=sqlite
    let mut query_types = prepare("select * from type")?;
    let mut rows = query_types.query(params![])?;
    while let Some(row) = rows.next()? {
        let id: i64 = row.get("type_id")?;
        let name: String = row.get("name")?;
        let ty = match &*name {
            "new_id" => ArgType::NewId,
            "int" => ArgType::Int,
            "uint" => ArgType::Uint,
            "fixed" => ArgType::Fixed,
            "string" => ArgType::String,
            "object" => ArgType::Object,
            "array" => ArgType::Array,
            "fd" => ArgType::Fd,
            _ => bail!("Unknown arg type {}", name),
        };
        types.insert(id, ty);
    }
    // language=sqlite
    let query_description = prepare("select * from description where description_id = $1")?;
    let query_description = RefCell::new(query_description);
    let get_description = |id: i64| {
        query_description.borrow_mut().query_row(params![id], |r| {
            Ok(Description {
                summary: r.get("summary")?,
                body: r.get("body")?,
            })
        })
    };
    // language=sqlite
    let mut query_args =
        prepare("select a.* from arg a where a.message_id = $1 order by position")?;
    let mut get_args = |id: i64| {
        let mut args = vec![];
        let mut rows = query_args.query(params![id])?;
        while let Some(row) = rows.next()? {
            let name: String = row.get("name")?;
            let type_id: i64 = row.get("type_id")?;
            let ty = *types.get(&type_id).context("unknown type id")?;
            let summary: Option<String> = row.get("summary")?;
            let description_id: Option<i64> = row.get("description_id")?;
            let interface: Option<String> = row.get("interface_name")?;
            let allow_null: bool = row.get("allow_null")?;
            let enum_: Option<String> = row.get("enum_name")?;
            let description = description_id.map(&get_description).transpose()?;
            args.push(Arg {
                name,
                ty,
                summary,
                description,
                interface,
                allow_null,
                enum_,
            });
        }
        Ok::<_, Error>(args)
    };
    // language=sqlite
    let mut query_messages = prepare("select * from message where interface_id = $1")?;
    let mut get_messages = |members: &mut Vec<(i64, Member)>, id: i64| {
        let mut rows = query_messages.query(params![id])?;
        while let Some(row) = rows.next()? {
            let id: i64 = row.get("message_id")?;
            let name: String = row.get("name")?;
            let is_request: bool = row.get("is_request")?;
            let is_destructor: bool = row.get("is_destructor")?;
            let since: Option<u32> = row.get("since")?;
            let deprecated_since: Option<u32> = row.get("deprecated_since")?;
            let description_id: Option<i64> = row.get("description_id")?;
            let description = description_id.map(&get_description).transpose()?;
            let ty = is_destructor.then_some(MessageType::Destructor);
            members.push((
                id,
                Member {
                    name,
                    since,
                    deprecated_since,
                    description,
                    ty: MemberType::Message(Message {
                        is_request,
                        ty,
                        args: get_args(id)?,
                    }),
                },
            ));
        }
        Ok::<_, Error>(())
    };
    // language=sqlite
    let mut query_entries = prepare("select * from entry where enum_id = $1 order by entry_id")?;
    let mut get_entries = |id: i64| {
        let mut entries = vec![];
        let mut rows = query_entries.query(params![id])?;
        while let Some(row) = rows.next()? {
            let name: String = row.get("name")?;
            let value: String = row.get("value_str")?;
            let summary: Option<String> = row.get("summary")?;
            let since: Option<u32> = row.get("since")?;
            let deprecated_since: Option<u32> = row.get("deprecated_since")?;
            let description_id: Option<i64> = row.get("description_id")?;
            let description = description_id.map(&get_description).transpose()?;
            entries.push(Entry {
                name,
                value,
                summary,
                since,
                deprecated_since,
                description,
            });
        }
        Ok::<_, Error>(entries)
    };
    // language=sqlite
    let mut query_enums = prepare("select * from enum where interface_id = $1")?;
    let mut get_enums = |members: &mut Vec<(i64, Member)>, id: i64| {
        let mut rows = query_enums.query(params![id])?;
        while let Some(row) = rows.next()? {
            let id: i64 = row.get("enum_id")?;
            let name: String = row.get("name")?;
            let since: Option<u32> = row.get("since")?;
            let bitfield: bool = row.get("is_bitfield")?;
            let description_id: Option<i64> = row.get("description_id")?;
            let description = description_id.map(&get_description).transpose()?;
            members.push((
                id,
                Member {
                    name,
                    since,
                    deprecated_since: None,
                    description,
                    ty: MemberType::Enum(Enum {
                        bitfield,
                        entries: get_entries(id)?,
                    }),
                },
            ));
        }
        Ok::<_, Error>(())
    };
    // language=sqlite
    let mut query_interfaces =
        prepare("select * from interface where protocol_id = $1 order by interface_id")?;
    let mut get_interfaces = |id: i64| {
        let mut interfaces = vec![];
        let mut rows = query_interfaces.query(params![id])?;
        while let Some(row) = rows.next()? {
            let id: i64 = row.get("interface_id")?;
            let name: String = row.get("name")?;
            let version: u32 = row.get("version")?;
            let frozen: Option<bool> = row.get("is_frozen")?;
            let description_id: Option<i64> = row.get("description_id")?;
            let description = description_id.map(&get_description).transpose()?;
            let mut members = vec![];
            get_messages(&mut members, id)?;
            get_enums(&mut members, id)?;
            members.sort_by_key(|m| m.0);
            let members = members.into_iter().map(|m| m.1).collect();
            interfaces.push(Interface {
                name,
                version,
                frozen,
                description,
                members,
            });
        }
        Ok::<_, Error>(interfaces)
    };
    // language=sqlite
    let mut query_protocols =
        prepare("select p.* from repo r join protocol p using (repo_id) where r.name = $1")?;
    let mut get_protocols = |repo: &Repo| {
        let mut protocols = vec![];
        let mut rows = query_protocols.query(params![repo.name])?;
        let set: HashSet<_> = match repo.handling {
            Handling::AllowAll { block } => block.iter().copied().collect(),
            Handling::DenyAll { allow } => allow.iter().copied().collect(),
        };
        while let Some(row) = rows.next()? {
            let name: String = row.get("name")?;
            match repo.handling {
                Handling::AllowAll { .. } if set.contains(&*name) => continue,
                Handling::DenyAll { .. } if set.not_contains(&*name) => continue,
                _ => {}
            }
            let id: i64 = row.get("protocol_id")?;
            let path: String = row.get("path")?;
            let copyright: Option<String> = row.get("copyright")?;
            let description_id: Option<i64> = row.get("description_id")?;
            let url = Some(format!("{}{}", repo.root, path));
            let copyright = copyright.map(|body| Copyright { body });
            let description = description_id.map(&get_description).transpose()?;
            protocols.push(Protocol {
                name: name_regex.replace_all(&name, "_").into_owned(),
                url,
                copyright,
                description,
                interfaces: get_interfaces(id)?,
            });
        }
        let suite = Suite {
            name: repo.name.to_string(),
            protocols,
        };
        Ok::<_, Error>(suite)
    };
    let mut suites = vec![];
    for repo in REPOS {
        suites.push(get_protocols(repo)?);
    }
    Ok(suites)
}
