===description===
MissingPropertyType fires for static class properties that have no type declaration.
===file===
<?php
class Counter {
    public static $count;
    private static $instance;
}
===expect===
MissingPropertyType@3:4-3:24: Property Counter::$count has no type annotation
MissingPropertyType@4:4-4:28: Property Counter::$instance has no type annotation
