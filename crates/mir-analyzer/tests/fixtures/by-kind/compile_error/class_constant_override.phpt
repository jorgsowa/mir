===description===
A class constant cannot override a final constant or narrow the inherited visibility; private constants are not inherited.
===file===
<?php
interface Limits {
    final const MAX = 10;
    const MIN = 0;
}

class Base {
    protected const LABEL = 'base';
    private const SECRET = 1;
    final public const VERSION = 1;
}

class Child extends Base implements Limits {
    const MAX = 20;
//  ^^^^^^^^^^^^^^^ CompileError: Child::MAX cannot override final constant Limits::MAX [final-constant-override]
    protected const MIN = 1;
//  ^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Access level to Child::MIN must be public (as in interface Limits) [constant-access-level]
    private const LABEL = 'child';
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Access level to Child::LABEL must be protected (as in class Base) or weaker [constant-access-level]
    private const SECRET = 2;
    const VERSION = 2;
//  ^^^^^^^^^^^^^^^^^^ CompileError: Child::VERSION cannot override final constant Base::VERSION [final-constant-override]
}

enum Size implements Limits {
    case Small;
    private const MIN = 5;
//  ^^^^^^^^^^^^^^^^^^^^^^ CompileError: Access level to Size::MIN must be public (as in interface Limits) [constant-access-level]
}
