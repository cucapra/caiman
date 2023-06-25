# Proposed Paper Structure

As written by Dietrich, 6/25/23

I wanted to have some proposal for the sections / stuff in the paper so that we
agree what we're focusing on.  Not necessarily anything "final", but something
concrete about how I am currently thinking about it to discuss.

## Overview

0. Abstract
1. Introduction/Motivation
  * Heterogeneous domain justification
  * Separation of concerns between perf and correctness
  * Decomposability for performance experimentation
2. Background (I think we can say why we're "not X" here?  Lots to say that for)
  * GPU programming concerns / performance
  * Scheduling languages
  * Theoretical ties (might have to be short?  Maybe more later?)
3. Theoretical Foundation
  * Overview (non-detailed?) of semantics
  * What is being restricted / checked
  * Flow of languages
4. Practical Caiman
  * Value language
  * Scheduling
  * Explication?
5. Language Semantics
  * Formal typing rules (?)
  * Soundness proof (?)
  * Other proofs?
6. Result
  * Whatever, list of examples implemented, machine details, blah blah blah
  * Why wgpu I guess
  * Performance changes in choices perhaps?  Maybe this could be in the intro?
6. Related Work
  * Scheduling languages
  * Heterogeneous languages (?)
  * Theoretical basis
7. Conclusion