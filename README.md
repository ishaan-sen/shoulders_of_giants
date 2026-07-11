# Shoulders of Giants

#### A blazingly fast academic citation graph analysis package

Academic research papers cite other papers. This is a key mechanism by which humanity compounds its scientific and technical knowledge: previous discoveries are used by new researchers as a basis for their knowledge. Papers can only cite papers that existed before they were published; therefore, no two papers can cite each other (since citations can only flow backward in time). This forms a directed acyclic graph of papers (represented as nodes), with all citations (represented as graph edges) flowing backward in time. It is useful to have access to this sort of a structure when mapping out a field of study. 

Mapping fields of study is important because it enables researchers to quickly understand the sort of work that has been done in a particular area of research before embarking on additional research studies. Automation tools for this, like the proposed DAG generator, would help simplify the literature review process. 

Features:
- Given a paper, list all of its incoming and outgoing edges and the papers they point to/from
- Given 2 papers, find the earliest common descendant
- Given 2 papers, find the last common ancestor 


Dataset: https://www.kaggle.com/datasets/nechbamohammed/research-papers-dataset 

