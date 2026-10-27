===description===
PHP 8.0's grammar takes one visibility before a promoted parameter, so a second modifier is a syntax error there (8.1 reports it at compile time); a repeated `readonly` is already `version-too-low`.
===config===
<mir>
  <phpVersion>8.0</phpVersion>
</mir>
===file===
<?php
class A {
    public function __construct(public private int $a, private Private int $b) {}
//                              ^^^^^^ ParseError: syntax error, unexpected token "private", expecting variable [multiple-visibility-modifiers]
//                                                     ^^^^^^^ ParseError: syntax error, unexpected token "private", expecting variable [multiple-visibility-modifiers]
}
class B {
    public private int $c = 1;
//  ^^^^^^^^^^^^^^ CompileError: Multiple access type modifiers are not allowed [multiple-visibility-modifiers]
}
