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
 */
function process($items): void {
    echo $items;
}
===expect===
InvalidDocblock@3:3-3:24: Invalid docblock: @param has empty generic type parameter in `array<>`
