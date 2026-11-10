===description===
A `Foo::*` docblock type names string constants, so it is a valid dynamic member name.
===file===
<?php
final class LogLevel
{
    public const ERROR = 'error';
    public const DEBUG = 'debug';
}

final class Logger
{
    public string $error = '';

    public function error(string $m): void
    {
        echo $m;
    }

    public function debug(string $m): void
    {
        echo $m;
    }
}

final class Handler
{
    /** @return LogLevel::* */
    public function level()
    {
        return LogLevel::ERROR;
    }

    public function run(Logger $logger): void
    {
        $level = $this->level();
        $logger->{$level}('x');
        echo $logger->{$level};
    }
}
