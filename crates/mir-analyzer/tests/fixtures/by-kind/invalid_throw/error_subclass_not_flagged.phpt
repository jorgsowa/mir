===description===
Throwing \TypeError (extends \Error) does not fire InvalidThrow
===config===
suppress=MissingThrowsDocblock
===file===
<?php
throw new \TypeError('type mismatch');
===expect===
