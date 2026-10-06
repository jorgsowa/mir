===description===
An enum that implements a deprecated interface should trigger DeprecatedInterface
===file===
<?php

/** @deprecated use NewStatus instead */
interface StatusInterface {}

enum Status: string implements StatusInterface {
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ DeprecatedInterface: Interface StatusInterface is deprecated: use NewStatus instead
    case Active = 'active';
}
