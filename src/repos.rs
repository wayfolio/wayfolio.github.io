use Handling::*;

pub struct Repo {
    pub name: &'static str,
    pub root: &'static str,
    pub handling: Handling,
}

pub enum Handling {
    AllowAll { block: &'static [&'static str] },
    DenyAll { allow: &'static [&'static str] },
}

pub static REPOS: &[Repo] = &[
    Repo {
        name: "wayland",
        root: "https://gitlab.freedesktop.org/wayland/wayland/-/blob/main/",
        // Should only contain the one protocol. Exclude other protocols in case they are
        // included for dev purposes.
        handling: DenyAll {
            allow: &["wayland"],
        },
    },
    Repo {
        name: "wayland-protocols",
        root: "https://gitlab.freedesktop.org/wayland/wayland-protocols/-/blob/main/",
        // These protocols are deprecated.
        handling: AllowAll {
            block: &[
                "linux_dmabuf_unstable_v1",
                "tablet_unstable_v1",
                "tablet_unstable_v2",
                "xdg_foreign_unstable_v1",
                "xdg_shell_unstable_v5",
                "xdg_shell_unstable_v6",
            ],
        },
    },
    Repo {
        name: "jay-protocols",
        root: "https://github.com/mahkoh/jay-protocols/blob/master/",
        handling: AllowAll { block: &[] },
    },
    Repo {
        name: "cosmic-protocols",
        root: "https://github.com/pop-os/cosmic-protocols/blob/main/",
        handling: AllowAll { block: &[] },
    },
    Repo {
        name: "external",
        root: "https://github.com/mahkoh/wayland-db-external/blob/master/",
        handling: AllowAll { block: &[] },
    },
    Repo {
        name: "hyprland-protocols",
        root: "https://github.com/hyprwm/hyprland-protocols/blob/main/",
        handling: AllowAll { block: &[] },
    },
    Repo {
        name: "river",
        root: "https://codeberg.org/river/river/src/branch/main/",
        handling: AllowAll { block: &[] },
    },
    Repo {
        name: "weston",
        root: "https://gitlab.freedesktop.org/wayland/weston/-/blob/main/",
        // Not namespaced.
        handling: AllowAll {
            block: &["text-cursor-position"],
        },
    },
    Repo {
        name: "wlr-protocols",
        root: "https://gitlab.freedesktop.org/wlroots/wlr-protocols/-/blob/master/",
        handling: AllowAll { block: &[] },
    },
    Repo {
        name: "treeland-protocols",
        root: "https://github.com/linuxdeepin/treeland-protocols/blob/master/",
        handling: AllowAll { block: &[] },
    },
    Repo {
        name: "plasma-wayland-protocols",
        root: "https://invent.kde.org/libraries/plasma-wayland-protocols/-/blob/master/",
        // KDE doesn't namespace their protocols. Can't include most of them.
        handling: DenyAll {
            allow: &[
                "kde_external_brightness_v1",
                "kde_lockscreen_overlay_v1",
                "kde_output_device_v2",
                "kde_output_management_v2",
                "kde_output_order_v1",
                "kde_primary_output_v1",
                "kde_screen_edge_v1",
                "org_kde_kwin_outputdevice",
                "org_kde_plasma_virtual_desktop",
                "plasma_shell",
                "plasma_window_management",
                "zkde_screencast_unstable_v1",
            ],
        },
    },
];
