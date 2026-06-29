===description===
Custom exception extending a multi-level hierarchy does not fire InvalidThrow
===file===
<?php
class DomainException extends \LogicException {}
class NotFoundException extends DomainException {}
class UserNotFoundException extends NotFoundException {}

throw new UserNotFoundException('user not found');
===expect===
