===description===
Correct case class name in catch clause is not reported.
===file===
<?php
class AppException extends \RuntimeException {}
try {
    throw new AppException("err");
} catch (AppException $e) {
}
===expect===
