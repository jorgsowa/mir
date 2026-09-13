===description===
Legacy `resource` property types are not undefined classes.
===config===
suppress=MissingConstructor,UnusedClass,UnusedProperty
===file===
<?php
class HandleBox {
    public resource $handle;
}
===expect===
