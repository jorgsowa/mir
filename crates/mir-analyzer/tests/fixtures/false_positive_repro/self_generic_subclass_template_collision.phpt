===description===
A self-generic subclass reusing the same conventional template letter
(`T`) as a `@extends`-fixed ancestor no longer collides: the subclass's
own `@var T`/method-param `T` resolves to the subclass's own binding,
and the ancestor's `@var T`/method's `T` resolves to the ancestor's
`@extends`-fixed binding — previously a single flat name-keyed map let
whichever merge direction happened to run last silently clobber the
other.
===config===
suppress=UnusedVariable,MissingConstructor,UnusedParam,MissingPropertyType
===file===
<?php
/**
 * @template T
 */
class Box {
    /** @var T */
    public $value;

    /** @return T */
    public function getValue() {
        return $this->value;
    }
}

/**
 * @template T
 * @extends Box<int>
 */
class Wrapper extends Box {
    /** @var T */
    public $extra;

    /** @return T */
    public function getExtra() {
        return $this->extra;
    }
}

/** @param Wrapper<string> $w */
function propertyAccess(Wrapper $w): void {
    takesString($w->extra);
    takesInt($w->value);
}

/** @param Wrapper<string> $w */
function methodCall(Wrapper $w): void {
    takesString($w->getExtra());
    takesInt($w->getValue());
}

function takesString(string $s): void {}
function takesInt(int $i): void {}
===expect===
