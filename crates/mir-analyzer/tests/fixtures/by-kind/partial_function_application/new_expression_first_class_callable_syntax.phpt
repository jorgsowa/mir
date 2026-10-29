===description===
`new Foo(...)`, `new class(...)` and `#[Attr(...)]` are compile errors; the `...` is not an empty argument list, so no `TooFewArguments` is reported next to them.
===config===
<mir>
  <phpVersion>8.5</phpVersion>
</mir>
===file===
<?php
final class Foo { public function __construct(public int $a) {} }
#[Attribute]
final class Attr { public function __construct(public int $a) {} }
#[Attr(...)]
//    ^^^^^ CompileError: Cannot create Closure as attribute argument [attribute-argument-closure]
function g(): void {}
function f(): Foo { return new Foo(...); }
//                                ^^^^^ CompileError: Cannot create Closure for new expression [closure-for-new]
function h(): object { return new class(...) { public function __construct(public int $a) {} }; }
//                                     ^^^^^ CompileError: Cannot create Closure for new expression [closure-for-new]
