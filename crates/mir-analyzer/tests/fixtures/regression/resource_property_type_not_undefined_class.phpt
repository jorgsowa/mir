===description===
Legacy native `resource` in a property type position must not be treated as an
undefined class while mir reports the underlying declaration error.
===config===
suppress=MissingConstructor,UnusedClass,UnusedProperty
===file===
<?php
class HandleBox {
    public resource $handle;
}
===expect===
