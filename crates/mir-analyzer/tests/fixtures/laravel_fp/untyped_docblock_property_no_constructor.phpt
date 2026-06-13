===description===
Laravel FP (laravel/framework): a property with only a docblock @var (no native
type) and no constructor is not "uninitialized" in PHP (untyped props default to
null), but mir's missing-constructor check uses the docblock type and emits
MissingConstructor (e.g. base Grammar::$connection). Ignored pending fix —
see ROADMAP §1.4 (class.rs uninit check should use native type).
===ignore===
===config===
suppress=MissingPropertyType,MissingClosureReturnType,UnusedParam,UnusedVariable,UnusedProperty
===file===
<?php
class Connection {}
class Grammar {
    /** @var Connection */
    protected $connection;
}
===expect===
