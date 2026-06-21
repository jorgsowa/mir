===description===
An enum that implements a deprecated interface should trigger DeprecatedInterface
===file===
<?php

/** @deprecated use NewStatus instead */
interface StatusInterface {}

enum Status: string implements StatusInterface {
    case Active = 'active';
}

===expect===
DeprecatedInterface@6:0-6:48: Interface StatusInterface is deprecated: use NewStatus instead
