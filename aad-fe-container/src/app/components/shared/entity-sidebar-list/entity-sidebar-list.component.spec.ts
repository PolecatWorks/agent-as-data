import { ComponentFixture, TestBed } from '@angular/core/testing';

import { EntitySidebarListComponent } from './entity-sidebar-list.component';

describe('EntitySidebarListComponent', () => {
  let component: EntitySidebarListComponent;
  let fixture: ComponentFixture<EntitySidebarListComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [EntitySidebarListComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(EntitySidebarListComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
