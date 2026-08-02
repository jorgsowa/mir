===description===
Same as param_bare_closure_still_not_flagged.phpt, but for a top-level
function rather than a class method — the two go through entirely separate
code paths (a method reuses its already-collected/resolved param type, a
free function re-parses and re-resolves the raw `@param` docblock in
body_analysis/functions.rs via crate::db::resolve_docblock_type_name, which
needed its own exemption for real global builtins).
===config===
suppress=UnusedParam
===file===
<?php
namespace App;

/**
 * @param Closure $callback
 */
function apply($callback): void {
}
===expect===
