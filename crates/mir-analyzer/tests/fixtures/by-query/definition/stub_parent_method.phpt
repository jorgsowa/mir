===description===
A method inherited from a built-in class lands in the stubs.
===cursor===
definition
===file===
<?php
final class AppError extends RuntimeException {}
echo (new AppError())->getMes<CURSOR>sage();
===expect===
stubs/Core/Core_c.php@335:4-336:49
