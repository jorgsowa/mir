===description===
No ReadonlyPropertyAssignment when initializing readonly in a non-constructor method of the declaring class
===config===
<mir>
  <issueHandlers>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo {
    public readonly string $name;

    public function init(string $name): void {
        $this->name = $name;
    }
}
===expect===
