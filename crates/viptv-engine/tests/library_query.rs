#![cfg(feature="runtime")]
use viptv_engine::{library::*,profile::ProfileConfiguration,store::Store,simkl,Error};
use serde_json::json;
#[test]
fn library_filters_page_real_data_and_preserve_unknown_personal_ratings_dates_and_counts(){
 let scope=ProfileConfiguration::local("local".into(),"one".into(),"One".into()).scope;
 let entry=|id,name:&str,genre:&str,added,watched|{let mut card=simkl::card(&json!({"title":name,"ids":{"simkl":id},"genres":[genre],"ratings":{"imdb":{"rating":9.7,"votes":100}}}),"movie").unwrap();card.added_at_millis=added;let mut statistics=LibraryStatistics::empty(card.identity.clone());statistics.watched=watched;statistics.user_rating=None;Entry{card,statistics}};
 let sources=vec![LibrarySource{id:"favorites".into(),label:"Saved titles".into(),availability:viptv_engine::capabilities::Availability::Available,reason:None}];let mut query=LibraryQuery::default();query.sort="name_desc".into();query.show="not_watched".into();query.genre=Some("Drama".into());
 let result=project(scope.clone(),0,query.clone(),vec![entry(1,"Alpha","Drama",Some(50),true),entry(2,"Zulu","Drama",None,false),entry(3,"Beta","Comedy",Some(100),false)],sources.clone(),200,None).unwrap();assert_eq!(result.cards.len(),1);assert_eq!(result.cards[0].title,"Zulu");assert!(result.statistics[0].user_rating.is_none());assert!(result.controls.iter().all(|c|c.id!="rated"&&c.id!="status"));
 query.rated=Some("rated_8_plus".into());assert!(matches!(project(scope,0,query,vec![],sources,200,None),Err(Error::InvalidQuery)));
}
#[test]
fn observed_native_completions_increment_once_per_rewatch_and_stay_profile_scoped(){
 let mut store=Store::memory().unwrap();let a=ProfileConfiguration::local("local".into(),"a".into(),"A".into());let b=ProfileConfiguration::local("local".into(),"b".into(),"B".into());store.save_profile(&a,None).unwrap();store.save_profile(&b,None).unwrap();let card=simkl::card(&json!({"title":"Title","ids":{"simkl":1}}),"movie").unwrap();store.save_favorite(&a.scope,&card).unwrap();store.save_favorite(&b.scope,&card).unwrap();
 for (position,now) in [(10000,1),(95000,2),(99000,3),(10000,4),(96000,5)]{store.record_playback(&a.scope,&card,position,100000,now).unwrap();}
 let result=store.library_entries(&a.scope).unwrap();assert_eq!(result[0].statistics.times_watched,Some(2));assert!(result[0].statistics.watched);assert_eq!(result[0].statistics.last_watched_millis,Some(5));let other=store.library_entries(&b.scope).unwrap();assert_eq!(other[0].statistics.times_watched,Some(0));assert!(!other[0].statistics.watched);
}
