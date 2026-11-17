===description===
From PHP 8.6 #[\Override] on a class constant or enum case needs a public or protected constant of that name in a parent class or interface.
===config===
<mir>
  <phpVersion>8.6</phpVersion>
</mir>
===file===
<?php
interface HasLabel {
    const LABEL = 'base';
}

class Base {
    public const A = 1;
    protected const B = 2;
    private const C = 3;
}

class Middle extends Base {}

class Child extends Middle implements HasLabel {
    #[\Override]
    public const A = 10;
    #[\Override]
    protected const B = 20;
    #[\Override]
    const LABEL = 'child';
    #[\Override]
    private const C = 30;
//  ^^^^^^^^^^^^^^^^^^^^^ InvalidOverride: Child::C has #[\Override] attribute, but no matching parent constant exists
    #[\Override]
    const MISSING = 40;
//  ^^^^^^^^^^^^^^^^^^^ InvalidOverride: Child::MISSING has #[\Override] attribute, but no matching parent constant exists
}

interface Extended extends HasLabel {
    #[\Override]
    const LABEL = 'extended';
}

enum Status implements HasLabel {
    #[\Override]
    case LABEL;
    #[\Override]
    case Other;
//  ^^^^^^^^^^^ InvalidOverride: Status::Other has #[\Override] attribute, but no matching parent constant exists
}

class UnknownParent extends NotDefinedAnywhere {
//                          ^^^^^^^^^^^^^^^^^^ UndefinedClass: Class NotDefinedAnywhere does not exist
    #[\Override]
    const X = 1;
}
