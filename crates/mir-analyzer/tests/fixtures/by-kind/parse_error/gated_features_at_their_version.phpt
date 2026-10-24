===description===
Each gated feature is accepted at the version that introduces it.
===config===
<mir>
  <phpVersion>8.3</phpVersion>
</mir>
===file===
<?php
enum Suit { case Hearts; }
readonly class A {
    const string NAME = "a";
    public function __construct(public int $x) {}
    public function m(int $x): string { return match ($x) { 1 => "a", default => "b" }; }
}
function f(?\DateTime $d): ?string { return $d?->format("Y"); }
