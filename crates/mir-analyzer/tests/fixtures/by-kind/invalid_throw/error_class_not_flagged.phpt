===description===
Throwing \Error does not fire InvalidThrow — Error implements Throwable
===config===
suppress=MissingThrowsDocblock
===file===
<?php
throw new \Error('fatal');
===expect===
