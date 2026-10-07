===description===
A quoted literal in a docblock type may contain type syntax (`>`, `<`, `,`,
`:`, `{`, `(`, `&`) without ending a generic, shape or callable early.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param array{op?: '->'|'::', next: int} $shape
 * @param list<'a,b'|'c>d'> $list
 * @param array<'k:v', '{x}'> $map
 * @param array{sep: '(', join: '&'} $chars
 * @param callable('<'): '>' $cb
 * @return array{arrow: '->', rest: int}
 */
function quoted(array $shape, array $list, array $map, array $chars, callable $cb): array {
    /** @mir-check $shape is array{op?: "->"|"::", next: int} */
    /** @mir-check $list is list<"a,b"|"c>d"> */
    /** @mir-check $map is array<"k:v", "{x}"> */
    /** @mir-check $chars is array{sep: "(", join: "&"} */
    $r = $cb('<');
    /** @mir-check $r is ">" */
    /** @var array{op: '->', n: int} $local */
    $local = ['op' => '->', 'n' => 1];
    /** @mir-check $local is array{op: "->", n: int} */
    return ['arrow' => '->', 'rest' => $shape['next']];
}
