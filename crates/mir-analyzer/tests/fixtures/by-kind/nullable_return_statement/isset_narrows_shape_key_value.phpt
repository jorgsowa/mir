===description===
`isset($arr['key'])` narrows that key's own value type inside the guard,
not just the base array — a shape property typed string|null returns
provably non-null after the check
===config===
<mir>
  <issueHandlers>
    <MissingParamType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param array{a: string|null} $arr */
function test(array $arr): string {
    if (isset($arr['a'])) {
        return $arr['a'];
    }
    return '';
}
===expect===
