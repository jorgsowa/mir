===description===
Imports chain through intermediate classes (A from B, B from C), mutual
imports don't loop, and an alias the source lacks degrades to mixed.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:c.php===
<?php
namespace N;

/** @psalm-type Id = int */
class C {}
===file:b.php===
<?php
namespace N;

/**
 * @psalm-import-type Id from C
 * @psalm-import-type Back from A
 */
class B {}
===file:a.php===
<?php
namespace N;

/**
 * @psalm-import-type Id from B
 * @psalm-import-type Missing from C
 * @psalm-type Back = string
 */
class A {
    /** @param Id $id */
    public function id($id): void {
        /** @mir-check $id is int */
        echo 1;
    }

    /** @param Missing $m */
    public function missing($m): void {
        /** @mir-check $m is mixed */
        echo 1;
    }
}
===expect===
