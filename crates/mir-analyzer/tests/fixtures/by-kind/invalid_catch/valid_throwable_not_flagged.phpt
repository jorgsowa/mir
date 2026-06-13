===description===
InvalidCatch does NOT fire for classes that do implement Throwable.
===config===
suppress=UnusedVariable
===file===
<?php
class AppException extends RuntimeException {}

try {
    throw new AppException("fail");
} catch (AppException $e) {
    echo $e->getMessage();
} catch (\Exception $e) {
    echo "generic";
}

===expect===
