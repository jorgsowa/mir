===description===
MissingConstructor does NOT fire when every property has a default value.
===file===
<?php
class WithDefaults {
    public string $name = "default";
    public int $count = 0;
}

new WithDefaults();

===expect===
