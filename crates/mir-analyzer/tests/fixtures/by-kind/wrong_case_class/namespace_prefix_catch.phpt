===description===
Wrong case in namespace prefix segment of a catch clause is reported.
===config===
suppress=UnusedVariable
===file===
<?php
namespace MyApp\Exceptions;
class ServiceException extends \RuntimeException {}

namespace Client;
try {
} catch (\myapp\exceptions\ServiceException $e) {
}
===expect===
WrongCaseClass@7:10-7:44: Class name 'myapp\exceptions\ServiceException' has incorrect casing; use 'MyApp\Exceptions\ServiceException'
