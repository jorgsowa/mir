===description===
`new Wrap([])` infers `Wrap<array{}>`; an empty array literal is a valid empty
list, so it must satisfy a declared `Wrap<list<AssetId>>` return type.
===config===
php_version=8.4
suppress=UnusedParam
===file===
<?php
namespace Demo;
use Demo\Ids\AssetId;
/**
 * @template T
 */
final class Wrap {
    /** @param T $v */
    public function __construct(public mixed $v) {}
}
/** @param Wrap<list<AssetId>> $w */
function takes(Wrap $w): void {}
/** @return Wrap<list<\Demo\Ids\AssetId>> */
function make(): Wrap { return new Wrap([]); }
function run(): void {
    // expect: InvalidArgument short-name AssetId vs FQN Demo\Ids\AssetId
    takes(make());
}
===expect===
