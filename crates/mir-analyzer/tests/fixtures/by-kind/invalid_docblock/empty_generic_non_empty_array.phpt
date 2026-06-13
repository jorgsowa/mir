===description===
empty generic non-empty-array in class property
===config===
suppress=MissingPropertyType
===file===
<?php
class Container {
    /**
     * @var non-empty-array<> $items
     */
    private $items = [];
}
===expect===
InvalidDocblock@3:0-3:0: Invalid docblock: @var has empty generic type parameter in `non-empty-array<>`
