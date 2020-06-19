# Burnout
**Anonymously keep track of your team's emotional health**

Burnout is a tool designed to make the development and hosting of secure, anonymous, emotional
health tracking tooling easy for you and your teams. It is built around a well defined API
specification which provides the ability to report emotional health on a wide range of metrics
to suit your use case.

This particular component is a batteries-excluded platform for managing team membership and
data persistence. It is intended to be used in conjunction with tailored user interfaces which
suit your team's needs.

Some suggested ways of reporting emotional health are a simple "Happy/Sad" indicator which
reports a `happy_sad` metric with a value of `+1` or `-1` respectively. Prompting your team
(automatically) on a regular basis to answer this can give an indication of whether things
are trending well, or whether you need to fix something.

Another good approach is to use [burnoutindex.org](https://burnoutindex.org) to get a more
specific indicator of burnout level. This takes some more time, so you should do it less
frequently, but it can be a good way to keep a heart-beat on your team's burnout level.

## Anonymity
This tool is designed to anonymize reports and will not keep track of who submitted what.
In smaller teams this may not be enough to prevent identification and if there is a risk
that identifying the user may lead to repercussions, perhaps you've got larger problems
to deal with in your team.