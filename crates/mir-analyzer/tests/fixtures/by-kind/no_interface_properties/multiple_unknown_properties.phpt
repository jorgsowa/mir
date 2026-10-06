===description===
NoInterfaceProperties fires once per distinct unknown property access on the
same sealed interface.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @property string $name
 * @seal-properties
 */
interface Sealed {
    /** @return mixed */
    public function __get(string $key);
    /** @param mixed $value */
    public function __set(string $key, $value): void;
}

function readMultiple(Sealed $s): void {
    $a = $s->name;
    $b = $s->age;
//           ^^^ NoInterfaceProperties: Property $age is not defined on this interface
    $s->role = "admin";
//  ^^^^^^^^^^^^^^^^^^ NoInterfaceProperties: Property $role is not defined on this interface
}
