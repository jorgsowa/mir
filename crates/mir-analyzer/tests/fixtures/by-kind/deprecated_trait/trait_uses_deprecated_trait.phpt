===description===
A trait that uses a deprecated trait should trigger DeprecatedTrait
===file===
<?php

/** @deprecated Use NewLogger instead */
trait DeprecatedLogger {}

trait ConsumerTrait {
//<^^^^^^^^^^^^^^^^^^^^^ DeprecatedTrait: Trait DeprecatedLogger is deprecated: Use NewLogger instead
    use DeprecatedLogger;
}
