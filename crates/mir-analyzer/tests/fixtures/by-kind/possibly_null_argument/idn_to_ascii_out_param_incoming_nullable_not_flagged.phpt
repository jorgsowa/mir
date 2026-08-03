===description===
M6: idn_to_ascii()/idn_to_utf8()'s 4th param ($idna_info) is a pure
out-param — its stub now uses @param mixed + @param-out array, matching
the parse_str() precedent, so a caller passing a nullable/uninitialized
by-ref variable purely to receive the output isn't flagged
PossiblyNullArgument against the (irrelevant) incoming type.
===config===
suppress=MissingReturnType,UnusedParam
===file===
<?php
function wrap(string $domain, int $options, ?array &$info = []) {
    return idn_to_ascii($domain, $options, INTL_IDNA_VARIANT_UTS46, $info);
}
function f(string $domain): void {
    wrap($domain, 0, $info);
}
===expect===
