===description===
FALSE POSITIVE reproducer. Valid PHP: `new Wrap([])` should infer the declared `Wrap<list<AssetId>>`; the type argument is dropped to `array{}`.
mir 0.42.0 currently emits (the bug): InvalidReturnType@14:24-14:44: actual Demo\Wrap<array{}>
Expected: no issue. Remove ===ignore=== to activate once fixed.
===ignore===
===config===
php_version=8.4
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
