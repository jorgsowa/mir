===description===
`@phpstan-import-type X from` may name the source class on the next docblock line.
===file===
<?php
namespace A;

/**
 * @phpstan-type InfoList array<string, string>
 */
interface H {}

/**
 * @phpstan-import-type InfoList from
 *   \A\H
 */
class F {
    /** @return array<string, InfoList> */
    public function f(): array {
        return ['a' => ['b' => 'c']];
    }
}
