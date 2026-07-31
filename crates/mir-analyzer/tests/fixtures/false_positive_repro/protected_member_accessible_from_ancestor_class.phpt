===description===
FP-K3: PHP's protected-visibility check is symmetric over the class
hierarchy (Zend's zend_check_protected accepts either direction), but
property_inaccessible and the class-constant visibility checks only ever
tested caller-extends-owner. A method on an ANCESTOR class reaching into a
protected property/constant declared only on a DESCENDANT (via a
descendant-typed parameter) was wrongly flagged inaccessible.
===config===
suppress=MissingConstructor,UnusedParam
===file===
<?php

class Base {
    public function peekProperty(Child $c): int {
        return $c->offset;
    }

    public function peekConstantByLiteralName(): string {
        return Child::SECRET;
    }

    public function peekConstantThroughReceiver(Child $c): string {
        return $c::SECRET;
    }
}

class Child extends Base {
    protected int $offset = 0;
    protected const SECRET = 'hidden';
}
===expect===
