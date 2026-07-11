===description===
`!empty($arr['key'])` narrows that key's own value type inside the guard,
not just the base array — a shape property typed string|null returns
provably non-null (and non-empty-string) after the check, matching
`isset($arr['key'])`'s existing narrowing
===config===
suppress=MissingParamType
===file===
<?php
/** @param array{a?: string|null} $arr */
function test(array $arr): string {
    if (!empty($arr['a'])) {
        return $arr['a'];
    }
    return '';
}
===expect===
