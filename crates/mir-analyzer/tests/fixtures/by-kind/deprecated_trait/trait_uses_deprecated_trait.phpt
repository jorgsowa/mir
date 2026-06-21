===description===
A trait that uses a deprecated trait should trigger DeprecatedTrait
===file===
<?php

/** @deprecated Use NewLogger instead */
trait DeprecatedLogger {}

trait ConsumerTrait {
    use DeprecatedLogger;
}

===expect===
DeprecatedTrait@6:0-6:21: Trait DeprecatedLogger is deprecated: Use NewLogger instead
