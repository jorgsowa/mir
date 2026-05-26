===description===
empty generic array parameter
===file===
<?php
/**
 * @param array<> $items
 */
function process($items): void {
    echo $items;
}
===expect===
InvalidDocblock@2:0: Invalid docblock: @param has empty generic type parameter in `array<>`
