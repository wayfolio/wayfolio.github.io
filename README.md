# wayfolio.github.io

This repository contains the code used to generate https://wayfolio.github.io.

## Coverage

The generator uses

- https://github.com/mahkoh/wayland-db and
- https://github.com/wayfolio/compositor-support

to get protocol definitions and compositor support.

Almost all protocols from wayland-db are included. See
[repos.rs](./src/repos.rs) for details about which protocols are excluded and
why.

## Schedule

The site is built automatically every 6 hours. Since wayland-db is also updated
every 6 hours, the site might lag behind by up to 12 hours.

## Building

To build the generator, you must first fetch a few submodules. You can run

```bash
./scripts/clone-repos.sh
```

to clone the repositories. If you are working on wayfolio, it might be simpler
to symlink your regular checkouts.

Since wayfolio does not follow semver, a particular revision is required.
This revision is automatically checked out by `clone-repos.sh`.

You can then build the site by running

```bash
./scripts/generate-page.sh
```

The page will be stored in the `page` directory. You can view it by running

```bash
cd page
caddy file-server --listen :8080
```

## License

The generator is free software licensed under the GNU General Public License
v3.0.
