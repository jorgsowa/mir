===description===
`class-string<T>` and `interface-string` reject text that is not a class name; a parameter that also takes `string` accepts it.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Handler {}
final class MailHandler implements Handler {}

/** @param class-string<Handler> $class */
function register(string $class): void {}

/** @param class-string<Handler>|string $id */
function resolve(string $id): void {}

/** @param class-string|null $class */
function optional(?string $class): void {}

register(MailHandler::class);
register('mail handler');
//       ^^^^^^^^^^^^^^ InvalidArgument: Argument $class of register() expects 'class-string<Handler>', got '"mail handler"'
resolve('mail handler');
optional(null);
optional('mail-handler');
//       ^^^^^^^^^^^^^^ InvalidArgument: Argument $class of optional() expects 'class-string|null', got '"mail-handler"'
