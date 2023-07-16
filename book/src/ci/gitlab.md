# GitLab CI/CD

Use this snippet to have a dedicated [job](https://docs.gitlab.com/ee/ci/jobs/)
in the [test stage of your pipeline](https://docs.gitlab.com/ee/ci/pipelines/):

```yml
msrv:
  stage: test
  image:
    name: foresterre/cargo-msrv:latest
    entrypoint: [""]
  before_script:
    - rustc --version
    - cargo --version
    - cargo msrv --version
  script:
    - cargo msrv --output-format minimal verify
```

**Note:** The empty `entrypoint` is necessary because the image has
`cargo-msrv` as its entrypoint. Since we want to run other commands, like
`cargo --version`, GitLab requires either an empty entrypoint or a shell.
