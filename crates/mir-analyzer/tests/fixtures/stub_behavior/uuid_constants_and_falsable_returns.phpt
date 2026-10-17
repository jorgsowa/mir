===description===
The uuid stub lived at `uuid/uuid.php` while PhpStormStubsMap.php points at
`uuid/uuid_c.php`, invented `UUID_TYPE_NAME_MD5`/`UUID_TYPE_NAME_SHA1` instead of
the real `UUID_TYPE_MD5`/`UUID_TYPE_SHA1`, lacked the `UUID_VARIANT_*` constants
and `uuid_generate_md5`/`uuid_generate_sha1`, and typed the failable
`uuid_time`/`uuid_mac`/`uuid_parse`/`uuid_unparse` without `|false`.
===file===
<?php

function v5(string $ns, string $name): string {
    return uuid_generate_sha1($ns, $name) . uuid_generate_md5($ns, $name);
}

function kind(string $uuid): bool {
    return uuid_type($uuid) === UUID_TYPE_MD5 || uuid_variant($uuid) === UUID_VARIANT_DCE;
}

function stamp(string $uuid): int {
    $t = uuid_time($uuid);
    /** @mir-check $t is int|false */
    $mac = uuid_mac($uuid);
    /** @mir-check $mac is string|false */
    return $t === false || $mac === false ? 0 : $t;
}
