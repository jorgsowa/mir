===description===
empty generic array parameter
===config===
<mir>
  <issueHandlers>
    <MissingParamType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param array<> $items
// ^^^^^^^^^^^^^^^^^^^^^ InvalidDocblock: Invalid docblock: @param has empty generic type parameter in `array<>`
 */
function process($items): void {
    echo $items;
}
