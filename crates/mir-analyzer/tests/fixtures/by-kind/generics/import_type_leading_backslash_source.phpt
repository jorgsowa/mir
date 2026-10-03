===description===
`@psalm-import-type ... from \Fqcn` resolves the source as a global name for
class, interface, trait and enum sources, not relative to the importer's namespace.
===config===
suppress=UnusedParam,UnusedVariable
===file:lib.php===
<?php
namespace Lib\Sub;

class Item {}

/** @psalm-type Items = array<int, Item> */
class C {}

/** @psalm-type Items = array<int, Item> */
interface I {}

/** @psalm-type Items = array<int, Item> */
trait T {}

/** @psalm-type Items = array<int, Item> */
enum E { case A; }
===file:app.php===
<?php
namespace App;

/**
 * @psalm-import-type Items as CI from \Lib\Sub\C
 * @psalm-import-type Items as II from \Lib\Sub\I
 * @psalm-import-type Items as TI from \Lib\Sub\T
 * @psalm-import-type Items as EI from \Lib\Sub\E
 */
class User {
    /** @param CI $i */
    public function c($i): void {
        /** @mir-check $i is array<int, Lib\Sub\Item> */
        echo 1;
    }
    /** @param II $i */
    public function i($i): void {
        /** @mir-check $i is array<int, Lib\Sub\Item> */
        echo 1;
    }
    /** @param TI $i */
    public function t($i): void {
        /** @mir-check $i is array<int, Lib\Sub\Item> */
        echo 1;
    }
    /** @param EI $i */
    public function e($i): void {
        /** @mir-check $i is array<int, Lib\Sub\Item> */
        echo 1;
    }
}
===expect===
