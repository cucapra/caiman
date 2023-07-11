# Proposed Paper Structure

As written by Dietrich, 6/25/23

I wanted to have some proposal for the sections / stuff in the paper so that we
agree what we're focusing on.  Not necessarily anything "final", but something
concrete about how I am currently thinking about it to discuss.

## Overview

* Abstract
* Introduction/Motivation
  * Heterogeneous domain justification
  * Separation of concerns between perf and correctness
  * Decomposability for performance experimentation
* Background (I think we can say why we're "not X" here?  Lots to say that for)
  * GPU programming concerns / performance
  * Scheduling languages
  * Theoretical ties (might have to be short?  Maybe more later?)
* Theoretical Foundation
  * Overview (non-detailed?) of semantics
  * What is being restricted / checked
  * Flow of languages
* Practical Caiman
  * Value language
  * Scheduling
  * Explication?
* Language Semantics
  * Formal typing rules (?)
  * Soundness proof (?)
  * Other proofs?
* Result
  * Whatever, list of examples implemented, machine details, blah blah blah
  * Why wgpu I guess
  * Performance changes in choices perhaps?  Maybe this could be in the intro?
* Related Work
  * Scheduling languages
  * Heterogeneous languages (?)
  * Theoretical basis
* Conclusion