===description===
interface_exists guard suppresses UndefinedClass on `interface ... extends`
===file===
<?php
if (!interface_exists(\Vendor\OptionalIface::class)) {
    throw new \RuntimeException('missing');
}
interface MyIface extends \Vendor\OptionalIface {}
===expect===
