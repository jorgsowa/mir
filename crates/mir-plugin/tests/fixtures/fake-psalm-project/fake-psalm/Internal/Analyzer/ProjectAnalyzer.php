<?php
namespace Psalm\Internal\Analyzer;

use Psalm\Codebase;
use Psalm\Config;
use Psalm\Internal\Provider\Providers;

class ProjectAnalyzer
{
    /** @var array<string, string> */
    private array $project_files = [];

    public function __construct(public Config $config, public Providers $providers)
    {
    }

    public function getCodebase(): Codebase
    {
        return new Codebase();
    }

    public function canReportIssues(string $file_path): bool
    {
        return isset($this->project_files[$file_path]);
    }
}
