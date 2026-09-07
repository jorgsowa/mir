===description===
Method-level @psalm-type merges with class-level aliases; both are available in the method
===config===
suppress=UnusedParam
===file===
<?php
namespace App;

/**
 * @psalm-type Size = "small"|"medium"|"large"
 */
class Inventory {
    /**
     * @psalm-type Color = "red"|"blue"|"green"
     * @param Size $size
     * @param Color $color
     * @return string
     */
    public function describe(string $size, string $color): string {
        return "$color $size";
    }
}

$inv = new Inventory();
$inv->describe("small", "red");
$inv->describe("large", "green");
===expect===
