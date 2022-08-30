- fetch redirects and control files async (do it after "caching is implemented")
- some paths don't redirect from `/unstable` at all: collect all the 404's: example `apache2`
  query: https://sources.debian.org/api/src/packagename/latest/
  https://sources.debian.org/api/src/apache2/2.4.54-2/debian/control/
  - get field `raw_url` for direct url with prefix
  - query field `checksum` for validation of control file (caching)
  for the latest package 
- ability for overrides
- some pkgs don't exist, but are included in the control files for example: `debhelper-compat`
- libraries themselves should be fuzzy matched
- automatically download popcon, and use the csv
- improve debcontrol error
- add config (for overrides)
- Care! Don't use it without the constrained inputs:
	- integrate `nix-locate` / `nix-index`
	https://github.com/Mic92/nix-index-database
- add unmatched inputs and outputs to `/outputs/{}.json`
- improve json output map:
	- check that were not overwriting insertions (maybe convert values to vec)
