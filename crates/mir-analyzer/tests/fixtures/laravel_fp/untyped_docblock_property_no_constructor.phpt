===description===
Regression (laravel/framework): a property with only a docblock @var (no native
type) and no constructor is not "uninitialized" in PHP (untyped props default to
null). The missing-constructor check now only counts native-typed properties, so
mir no longer emits MissingConstructor (e.g. base Grammar::$connection).
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
