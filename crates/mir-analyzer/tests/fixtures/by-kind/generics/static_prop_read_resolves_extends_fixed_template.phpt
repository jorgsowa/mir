===description===
Reading a static property (`self::$prop`, `Class::$prop`) resolves an
inherited `@extends Box<int>`-fixed ancestor template, same as an instance
property read and the write side already do — the read paths took `p.ty`
raw with no `inherited_template_bindings` substitution at all.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <MissingConstructor errorLevel="suppress"/>
    <MissingPropertyType errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template T */
abstract class Box {
    /** @var T */
    protected static $staticValue;
}

/** @extends Box<int> */
final class IntBox extends Box {
    public static function getViaSelf() {
        /** @mir-check self::$staticValue is int */
        $_ = self::$staticValue;
        return self::$staticValue;
    }

    public static function getViaClassName() {
        /** @mir-check IntBox::$staticValue is int */
        $_ = IntBox::$staticValue;
        return IntBox::$staticValue;
    }
}

/** @template T */
abstract class PlainBox {
    /** @var T */
    protected static $staticValue;
}

class NoFixedTemplate extends PlainBox {
    public static function getRaw() {
        // No `@extends` type args to resolve — legitimately stays the raw,
        // unsubstituted template.
        /** @mir-check self::$staticValue is T */
        $_ = self::$staticValue;
        return self::$staticValue;
    }
}
