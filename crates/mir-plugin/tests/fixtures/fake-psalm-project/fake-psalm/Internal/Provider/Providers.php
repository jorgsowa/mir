<?php
namespace Psalm\Internal\Provider;

class Providers
{
    public function __construct(public FileProvider $file_provider)
    {
    }
}
