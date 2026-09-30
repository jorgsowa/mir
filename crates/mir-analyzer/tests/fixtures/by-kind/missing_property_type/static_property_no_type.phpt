===description===
MissingPropertyType fires for static class properties that have no type declaration.
===file===
<?php
class Counter {
    public static $count;
//  ^^^^^^^^^^^^^^^^^^^^ MissingPropertyType: Property Counter::$count has no type annotation
    private static $instance;
//  ^^^^^^^^^^^^^^^^^^^^^^^^ MissingPropertyType: Property Counter::$instance has no type annotation
}
===expect===
