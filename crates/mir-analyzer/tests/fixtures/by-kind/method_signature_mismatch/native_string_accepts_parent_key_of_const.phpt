===description===
A native `string` param accepts every key a parent's `key-of<C::CONST>` allows.
===file===
<?php
class B { public const S = ['stable' => 0, 'dev' => 20]; }
interface I { /** @param key-of<B::S> $s */ public function set(string $s): void; }
final class C implements I { public function set(string $s): void {} }
