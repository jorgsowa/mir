===description===
empty generic non-empty-array in class property
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Container {
    /**
     * @var non-empty-array<> $items
//     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidDocblock: Invalid docblock: @var has empty generic type parameter in `non-empty-array<>`
     */
    private $items = [];
}
