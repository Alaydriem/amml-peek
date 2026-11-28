# The body of the ingame mise task: packages peek and modmanager, then runs them in game through amml's mod-neutral
# runner (`mise run ingame-run`, see docs/ingame.md in amml). The steps of the run are in scripts/ingame-steps.sh.
set -e
amml="${usage_amml:-${AMML_DIR:-../amml}}"
[ -f "$amml/scripts/ingame-run.sh" ] || { echo "no amml checkout at '$amml' (--amml, else AMML_DIR, else ../amml)"; exit 1; }
amml="$(cd "$amml" && pwd -P)"
modmanager="${usage_modmanager:-${AMML_MODMANAGER_DIR:-../modmanager}}"
[ -f "$modmanager/mise.toml" ] || { echo "no modmanager checkout at '$modmanager' (--modmanager, else AMML_MODMANAGER_DIR, else ../modmanager)"; exit 1; }
modmanager="$(cd "$modmanager" && pwd -P)"

mise run package
mise --cd "$modmanager" run package
args=(--instance "${usage_instance:?--instance is required}"
  --mod "$PWD/dist/@alaydriem/peek" --mod "$modmanager/dist/@amml/modmanager"
  --world-copy peek
  --steps "$PWD/scripts/ingame-steps.sh")
mise --cd "$amml" run ingame-run -- "${args[@]}"
